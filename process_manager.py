"""
Manages both the python and rust processes.
This script will start both processes and if requested also restart both processes.
"""


import os
import signal
import subprocess
import sys

rust_app = None
python_proc = None

def start_rust(rust_app_dir):
    global rust_app
    # Specify the path to the Rust application binary
    rust_app_binary = os.path.join(rust_app_dir, "dd_bot")

    rust_app = subprocess.Popen([rust_app_binary], cwd=rust_app_dir)
    print(f"Started Rust app with PID: {rust_app.pid}")

def start_python(python_main_dir):
    global python_proc
    python_proc = subprocess.Popen(["python", python_main_dir])
    print(f"Started Python app with PID: {python_proc.pid}")


def rust_shutdown():
    global rust_app
    if rust_app == None:
        print("Rust app not started")
        return
    os.kill(rust_app.pid, signal.SIGTERM)

    rust_app.wait()
    print("Rust app shutdown gracefully")

def python_shutdown():
    global python_proc
    if python_proc == None:
        print("Python app not started")
        return
    os.kill(python_proc.pid, signal.SIGTERM)

    rust_app.wait()
    print("Python app shutdown gracefully")

def restart_both():
    print('Restarting rust app...')
    rust_shutdown()
    print('Restarting python app...')
    python_shutdown()

    print("Gracefully shut down both")
    print("Starting up rust app")

    start_rust("./dd_bot/target/debug/")
    start_python("./discord_bot/main.py")

    print("All systems live!")



start_rust("./dd_bot/target/debug/")

start_python("./discord_bot/main.py")


def signal_handler(sig, frame):
    print('Shutting down...')
    rust_shutdown()
    python_shutdown()
    sys.exit(0)

# Register the signal handler for graceful shutdown
signal.signal(signal.SIGINT, signal_handler)


# Keep the script running to listen for signals
try:
    while True:
        pass
except KeyboardInterrupt:
    pass