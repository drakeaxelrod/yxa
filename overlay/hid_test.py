#!/usr/bin/env python3
"""Direct HID test - bypasses hidapi, reads raw from /dev/hidraw*"""

import os
import sys
import struct

# Force unbuffered output
sys.stdout.reconfigure(line_buffering=True)

def find_szr35_hidraw():
    """Find the SZR35 Raw HID interface"""
    for i in range(20):
        path = f"/dev/hidraw{i}"
        uevent = f"/sys/class/hidraw/hidraw{i}/device/uevent"
        if not os.path.exists(path):
            continue
        try:
            with open(uevent) as f:
                content = f.read()
                if "SZR35" in content:
                    # Check if this is the raw HID interface (usually input1 or input2)
                    if "input1" in content or "input2" in content:
                        return path, i
        except:
            pass
    return None, None

def main():
    path, num = find_szr35_hidraw()
    if not path:
        print("SZR35 not found!")
        print("\nAvailable hidraw devices:")
        for i in range(20):
            uevent = f"/sys/class/hidraw/hidraw{i}/device/uevent"
            if os.path.exists(uevent):
                with open(uevent) as f:
                    for line in f:
                        if "HID_NAME" in line:
                            print(f"  hidraw{i}: {line.strip()}")
        sys.exit(1)

    print(f"Found SZR35 at {path}")
    print("Requesting current layer state...")

    try:
        # Open for read/write
        fd = os.open(path, os.O_RDWR | os.O_NONBLOCK)

        # Send request for layer state (MSG_REQUEST_STATE = 0x00)
        request = bytes([0x00] + [0]*31)  # 32 byte packet
        try:
            os.write(fd, request)
            print("Sent layer request")
        except Exception as e:
            print(f"Write failed: {e}")

        print("\nWaiting for layer changes... (Ctrl+C to quit)")
        print("Switch layers on your keyboard to see updates.\n")

        import select
        while True:
            # Wait for data with timeout
            r, _, _ = select.select([fd], [], [], 0.5)
            if r:
                try:
                    data = os.read(fd, 64)
                    if data:
                        print(f"Received: {data.hex()}")
                        if data[0] == 0x01:  # MSG_LAYER_STATE
                            print(f"  -> Layer: {data[1]}")
                except BlockingIOError:
                    pass

    except PermissionError:
        print(f"Permission denied on {path}")
        print("Run: sudo chmod 666 /dev/hidraw*")
    except KeyboardInterrupt:
        print("\nDone")
    finally:
        try:
            os.close(fd)
        except:
            pass

if __name__ == "__main__":
    main()
