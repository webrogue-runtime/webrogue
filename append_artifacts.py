import sys

input_path = sys.argv[1]
artifacts_path = sys.argv[2]
output_path = sys.argv[3]

with open(output_path, "wb") as output_file:
    output_file.write(open(input_path, "rb").read())
    artifacts_file = open(artifacts_path, "rb").read()
    output_file.write(artifacts_file)
    output_file.write(len(artifacts_file).to_bytes(8, "little"))
