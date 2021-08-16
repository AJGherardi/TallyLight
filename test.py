import serial

ser = serial.Serial('/dev/ttyACM0')
ser.write(str.encode("prog_on\n"))
ser.write(str.encode("prev_off\n"))