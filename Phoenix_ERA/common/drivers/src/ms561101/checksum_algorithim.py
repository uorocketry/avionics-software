CK_A = 0
CK_B = 0


# This no work :(
byte_string = input("Enter the comma separated bytes: ").split(",")
n = len(byte_string)
buffer = []

for i in range(n):
    buffer.append(int(byte_string[i]))
print(f"{buffer}")

for byte in buffer:
    CK_A = (CK_A + byte) % 255
    CK_B = (CK_B + CK_A) % 255



print(f"CHECKSUM LOW {CK_A}, CHECKSUM HIGH {CK_B}")