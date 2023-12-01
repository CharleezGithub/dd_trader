"""
Manages both the python and rust processes.
This script will start both processes and if requested also restart both processes.
"""


import os
import signal
import subprocess
import sys
import time


rust_app = None
python_proc = None

def start_rust(rust_app_dir):
    global rust_app
    rust_app_binary = os.path.join(rust_app_dir, "dd_bot")
    # Open in a new command prompt window
    rust_app = subprocess.Popen(f'start cmd /c {rust_app_binary}', cwd=rust_app_dir, shell=True)
    print(f"Started Rust app with PID: {rust_app.pid}")

def start_python(python_main_dir):
    global python_proc
    # Open in a new command prompt window
    python_proc = subprocess.Popen(f'start cmd /c python {python_main_dir}', shell=True)
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
    print('Shutting down rust app...')
    rust_shutdown()
    print('Shutting down python app...')
    python_shutdown()

    print("Gracefully shut down both apps")
    print("Starting up again...")

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


def read_file_contents(path):
    with open(path, "r") as file:
        return file.read()


def file_has_changed(path, last_mod_time):
    try:
        current_mod_time = os.stat(path).st_mtime
        if current_mod_time != last_mod_time:
            contents = read_file_contents(path)
            return True, current_mod_time, contents
        else:
            return False, current_mod_time, None
    except FileNotFoundError:
        return None, None, None  # Indicate the file is not accessible


def monitor_file_changes(path_to_watch, interval=1):
    last_mod_time = os.stat(path_to_watch).st_mtime

    while True:
        changed, new_mod_time, contents = file_has_changed(path_to_watch, last_mod_time)
        if changed is None:  # File not found or inaccessible
            return  # Stop the generator
        if changed:
            yield contents  # Yield the new contents of the file
            last_mod_time = new_mod_time
        time.sleep(interval)

path_to_monitor = "shared/ipc_restart.txt"
polling_interval = 1  # seconds

# Every time the data in ipc_communication.txt is changed this will run again.
# It will run forever untill stopped.
for data in monitor_file_changes(path_to_monitor, polling_interval):
    if "restart request" == data:
        restart_both()