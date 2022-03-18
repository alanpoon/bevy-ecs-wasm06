#func (m *MessageEnvelope) Marshal() (dAtA []byte, err error)
#func (m *MessageEnvelope) MarshalTo(dAtA []byte) (int, error)
#func (m *MessageEnvelope) MarshalToSizedBuffer(dAtA []byte) (int, error)

import os
import shutil


for filename in os.listdir("src"):
  print("filename",filename)
  
  with open("../gen/go2/"+filename,"r") as f:
    foldername = filename.replace(".go","")
    lines = f.readlines()
    encodeVar = "encodeVarint"+filename.split(".pb")[0].capitalize()
    sov = "sov"+filename.split(".pb")[0].capitalize()
    skip = "skip"+filename.split(".pb")[0].capitalize()
    for j in lines:
      if "package" in j:
        packagename = j.split("package")[1].strip()
        foldername = j.split("package")[1].strip()+".pb"
        os.mkdir("../gen/go3/"+foldername)
        break
    with open("../gen/go3/"+foldername+"/"+filename,"w") as w:
      filter_out = False
      for j in lines:
        if "func" in j:
          filter_out = True
          for n in ["Marshal()"," Unmarshal(","MarshalTo","MarshalToSizedBuffer"," Size()",encodeVar,sov,skip]:
            if n in j:
              filter_out = False            
        if j.startswith("var") or j.startswith("type"):
          filter_out = False
        for n in ["var _ = proto.Marshal","proto.GoGoProtoPackageIsVersion3","var xxx","var fileDescriptor"]:
          if n in j:
            filter_out = True
          pass
        if "const (" in j:
          filter_out = True
        if filter_out ==False and 'io "io"' not in j and "github.com/gogo/protobuf/proto" not in j:
          j = j.replace("io.ErrUnexpectedEOF",'fmt.Errorf("ErrUnexpectedEOF")')
          w.write(j)