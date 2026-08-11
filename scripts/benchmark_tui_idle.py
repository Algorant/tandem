#!/usr/bin/env python3
"""Benchmark Tandem TUI idle CPU and Logs interaction through a fixed-size PTY.

CPU sampling uses Linux /proc. The script is dependency-free beyond Python 3.
"""

import argparse
import contextlib
import fcntl
import os
from pathlib import Path
import pty
import select
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time

ROWS = 46
COLS = 150
DEFAULT_COUNTS = (10, 50, 100, 250)
MAX_IDLE_CPU = 5.0
CLOCK_TICKS = os.sysconf("SC_CLK_TCK") if sys.platform.startswith("linux") else None


def write_workspace(root: Path, count: int) -> None:
    tandem = root / ".tandem"
    (tandem / "board").mkdir(parents=True, exist_ok=True)
    (tandem / "logs").mkdir(exist_ok=True)
    (tandem / "events").mkdir(exist_ok=True)
    (tandem / "tandem.md").write_text(
        """---
protocolVersion: "0.2.0"
type: workspace
title: "TUI idle benchmark"
states:
  - id: todo
    title: To Do
  - id: in-progress
    title: In Progress
  - id: validation
    title: Validation
---
""",
        encoding="utf-8",
    )
    for index in range(1, count + 1):
        (tandem / "logs" / f"task-{index}.md").write_text(
            f"""---
id: task-{index}
type: task
title: "Generated log {index}"
createdAt: "2026-01-01T00:00:00Z"
updatedAt: "2026-01-01T00:00:00Z"
completedAt: "2026-01-01T00:{index % 60:02d}:00Z"
completion:
  summary: "Generated benchmark item {index}"
  validation: "benchmark fixture"
---

# Generated body

Stable benchmark content.
""",
            encoding="utf-8",
        )


def prepare_workspace(target: Path, count: int) -> Path:
    target = target.expanduser().absolute()
    if os.path.lexists(target):
        raise FileExistsError(
            f"refusing to replace existing preview workspace: {target}; "
            "choose a new path or remove the intended benchmark fixture explicitly"
        )
    target.mkdir(parents=True)
    write_workspace(target, count)
    return target


def check_prepare_refusal() -> bool:
    with tempfile.TemporaryDirectory(prefix="tandem-prepare-refusal-") as temp:
        target = Path(temp)
        marker = target / "must-survive.txt"
        marker.write_text("user data\n", encoding="utf-8")
        try:
            prepare_workspace(target, 1)
        except FileExistsError:
            return marker.read_text(encoding="utf-8") == "user data\n"
        return False


def process_ticks(pid: int) -> int:
    stat = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8")
    fields = stat[stat.rfind(")") + 2 :].split()
    return int(fields[11]) + int(fields[12])


def drain(fd: int) -> int:
    total = 0
    while True:
        try:
            chunk = os.read(fd, 65536)
            if not chunk:
                return total
            total += len(chunk)
        except BlockingIOError:
            return total
        except OSError:
            return total


def wait_for_frame(fd: int, started: float, timeout: float) -> float | None:
    deadline = started + timeout
    saw_output = False
    while time.monotonic() < deadline:
        wait = min(0.03, deadline - time.monotonic())
        ready, _, _ = select.select([fd], [], [], wait)
        if not ready:
            if saw_output:
                return time.monotonic() - started
            continue
        saw_output = drain(fd) > 0 or saw_output
    return None


class TuiProcess:
    def __init__(self, binary: Path, workspace: Path):
        master, slave = pty.openpty()
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
        self.master = master
        self.process = subprocess.Popen(
            [str(binary), "tui"],
            cwd=workspace,
            stdin=slave,
            stdout=slave,
            stderr=slave,
            start_new_session=True,
        )
        os.close(slave)
        os.set_blocking(master, False)

    def settle(self, seconds: float) -> None:
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            drain(self.master)
            time.sleep(0.01)
        if self.process.poll() is not None:
            raise RuntimeError(f"TUI exited early with status {self.process.returncode}")

    def key(self, value: bytes) -> None:
        os.write(self.master, value)

    def close(self) -> None:
        with contextlib.suppress(OSError):
            self.key(b"q")
        try:
            self.process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(self.process.pid, signal.SIGTERM)
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait(timeout=2)
        with contextlib.suppress(OSError):
            os.close(self.master)


def sample_cpu(tui: TuiProcess, seconds: float) -> float:
    before = process_ticks(tui.process.pid)
    started = time.monotonic()
    deadline = started + seconds
    while time.monotonic() < deadline:
        drain(tui.master)
        time.sleep(0.01)
    elapsed = time.monotonic() - started
    after = process_ticks(tui.process.pid)
    return 100.0 * (after - before) / CLOCK_TICKS / elapsed


def benchmark_view(binary: Path, count: int, logs_view: bool, sample_seconds: float):
    with tempfile.TemporaryDirectory(prefix="tandem-tui-idle-") as temp:
        workspace = Path(temp)
        write_workspace(workspace, count)
        tui = TuiProcess(binary, workspace)
        try:
            tui.settle(1.5)
            if logs_view:
                tui.key(b"2")
                tui.settle(1.5)
            cpu = sample_cpu(tui, sample_seconds)
            interaction = None
            reload_latency = None
            if logs_view:
                interaction = wait_for_output_after_key(tui, b"j")
                log_path = workspace / ".tandem" / "logs" / f"task-{count}.md"
                drain(tui.master)
                started = time.monotonic()
                with log_path.open("a", encoding="utf-8") as handle:
                    handle.write("\n")
                reload_latency = wait_for_frame(tui.master, started, 1.2)
            return cpu, interaction, reload_latency
        finally:
            tui.close()


def wait_for_output_after_key(tui: TuiProcess, key: bytes) -> float | None:
    drain(tui.master)
    started = time.monotonic()
    tui.key(key)
    return wait_for_frame(tui.master, started, 1.0)


def machine_summary() -> str:
    model = "unknown"
    with contextlib.suppress(OSError, IndexError):
        for line in Path("/proc/cpuinfo").read_text(encoding="utf-8").splitlines():
            if line.startswith("model name"):
                model = line.split(":", 1)[1].strip()
                break
    return f"platform={sys.platform} kernel={os.uname().release} cpu={model} clock_ticks={CLOCK_TICKS}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("tandem/target/release/tandem"))
    parser.add_argument("--counts", default=",".join(map(str, DEFAULT_COUNTS)))
    parser.add_argument("--sample-seconds", type=float, default=4.0)
    parser.add_argument("--report-only", action="store_true")
    parser.add_argument("--prepare-workspace", type=Path)
    parser.add_argument("--prepare-count", type=int, default=250)
    parser.add_argument("--check-prepare-refusal", action="store_true")
    args = parser.parse_args()

    if args.check_prepare_refusal:
        if check_prepare_refusal():
            print("prepare-workspace refusal check=pass")
            return 0
        print("FAIL: prepare-workspace changed or replaced an existing directory", file=sys.stderr)
        return 1
    if args.prepare_workspace:
        try:
            target = prepare_workspace(args.prepare_workspace, args.prepare_count)
        except FileExistsError as error:
            print(f"error: {error}", file=sys.stderr)
            return 2
        print(target)
        return 0
    if not sys.platform.startswith("linux"):
        print("error: CPU assertions require Linux /proc; use --report-only for unsupported platforms", file=sys.stderr)
        return 2 if not args.report_only else 0

    binary = args.binary.resolve()
    if not binary.is_file():
        print(f"error: binary not found: {binary}", file=sys.stderr)
        return 2
    counts = tuple(int(value) for value in args.counts.split(","))
    print(f"binary={binary}")
    print(machine_summary())
    print(f"pty={COLS}x{ROWS} settle=1.5s sample={args.sample_seconds:.1f}s threshold={MAX_IDLE_CPU:.1f}%")
    print("logs,board_cpu_pct,logs_cpu_pct,input_ms,reload_ms")
    results = []
    for count in counts:
        board_cpu, _, _ = benchmark_view(binary, count, False, args.sample_seconds)
        logs_cpu, interaction, reload_latency = benchmark_view(binary, count, True, args.sample_seconds)
        results.append((count, board_cpu, logs_cpu, interaction, reload_latency))
        interaction_text = "timeout" if interaction is None else f"{interaction * 1000:.1f}"
        reload_text = "timeout" if reload_latency is None else f"{reload_latency * 1000:.1f}"
        print(f"{count},{board_cpu:.2f},{logs_cpu:.2f},{interaction_text},{reload_text}", flush=True)

    if args.report_only:
        print("result=report-only")
        return 0
    largest = max(results, key=lambda row: row[0])
    failures = []
    if largest[2] > MAX_IDLE_CPU:
        failures.append(f"Logs idle CPU {largest[2]:.2f}% exceeds {MAX_IDLE_CPU:.1f}% at {largest[0]} logs")
    if largest[3] is None or largest[3] > 0.25:
        failures.append("Logs selection redraw exceeded 250 ms")
    if largest[4] is None or largest[4] > 1.0:
        failures.append("external log change was not redrawn within 1 second")
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print("result=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
