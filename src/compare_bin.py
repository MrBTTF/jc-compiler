
src = []
with open("example/hello", "rb") as f:
     while (byte := f.read(1)):
        src.append(byte)

dst = []
with open("bin/hello", "rb") as f:
     while (byte := f.read(1)):
        dst.append(byte)

assert len(src) == len(dst)

for i in range(len(dst)):
    if src[i] != dst[i]:
        print(i)
        print(i)