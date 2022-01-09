import serial

ser = serial.Serial('/dev/ttyACM0')
ser.write(str.encode("p|100"))
ser.write(str.encode("P|100"))