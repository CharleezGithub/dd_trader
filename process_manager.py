import subprocess
import ctypes
import time
import os
import signal
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler
import sys

class RestartHandler(FileSystemEventHandler):
    def __init__(self):
        self.last_modified = time.time()

    def on_modified(self, event):
        if event.src_path == "shared\ipc_restart.txt":
            current_time = time.time()
            # Check if at least 2 seconds have passed since the last modification
            if current_time - self.last_modified > 10:
                print("Restart file modified. Restarting processes...")
                stop_processes()
                start_processes()
                self.last_modified = current_time

def start_processes():
    global process1, process2
    # Start each process in a new console
    process1 = subprocess.Popen(['cmd', '/c', 'cd dd_bot && cargo run'], creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)
    process2 = subprocess.Popen(['cmd', '/c', 'python discord_bot/main.py'], creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)

def send_ctrl_c(process):
    if os.name == 'nt':  # Windows
        # Send the Ctrl+C signal only to the process's console
        os.kill(process.pid, signal.CTRL_C_EVENT)
    else:  # Unix
        # Send SIGINT (Ctrl+C)
        os.kill(process.pid, signal.SIGINT)

def stop_processes():
    global process1, process2
    #send_ctrl_c(process1)
    os.kill(process1.pid, signal.CTRL_BREAK_EVENT)
    os.kill(process2.pid, signal.CTRL_BREAK_EVENT)

    # Wait for the processes to terminate
    process1.wait()
    process2.wait()

    # Optional: add a delay
    time.sleep(5)  # Delay for 5 seconds, adjust as needed


"""def is_admin():
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False"""


if __name__ == "__main__":
    """if not is_admin():
        # Re-run the program with admin rights
        ctypes.windll.shell32.ShellExecuteW(None, "runas", sys.executable, " ".join(sys.argv), None, 1)
        exit()"""

    start_processes()

    event_handler = RestartHandler()
    observer = Observer()
    observer.schedule(event_handler, path='shared', recursive=False)
    observer.start()

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()

    observer.join()
    stop_processes()
