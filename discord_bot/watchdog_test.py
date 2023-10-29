import time
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class MyHandler(FileSystemEventHandler):
    def on_any_event(self, event):
        print(f'Event type: {event.event_type} - path: {event.src_path}')

if __name__ == "__main__":
    path = "shared/ipc_communication.txt"  # Adjust this to the directory you are watching
    print(f"Watching for changes on: {path}")
    event_handler = MyHandler()
    observer = Observer()
    observer.schedule(event_handler, path, recursive=True)
    observer.start()

    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    except Exception as e:
        print(f"An error occurred: {e}")
    finally:
        observer.join()
