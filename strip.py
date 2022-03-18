#func (m *MessageEnvelope) Marshal() (dAtA []byte, err error)
#func (m *MessageEnvelope) MarshalTo(dAtA []byte) (int, error)
#func (m *MessageEnvelope) MarshalToSizedBuffer(dAtA []byte) (int, error)

import os
import shutil

import pathlib

for path, subdirs, files in os.walk("src"):
    for name in files:
        f = os.path.join(path, name)
        with open(f,"r") as r:
          pathlib.Path(os.path.join("gen",path)).mkdir(parents=True, exist_ok=True)
          lines = r.readlines()
          with open(os.path.join("gen",f),"w") as w:
            restricted = ""
            watch_restricted = False
            for j in lines:
              filter_out = False
              if "use bevy_utils::{tracing::info, HashMap, HashSet};" in j or "use bevy_utils::{tracing::warn, HashMap, HashSet};" in j:
                j = "use std::collections::{HashMap,HashSet};\n"
              if "use bevy_utils::{AHasher, HashMap};" in j:
                j = "use std::collections::{HashMap,hash_map::DefaultHasher};\n"
              if "use bevy_utils::{tracing::info, HashMap, HashSet};" in j:
                j = "use std::collections::{HashMap,hash_map::DefaultHasher,HashSet};\nuse std::hash::{Hasher};\n"
              if "use bevy_utils::tracing::" in j:
                filter_out =True
              if "use bevy_utils::HashMap" in j:
                j = "use std::collections::HashMap;\n"
              if "warn!" in j or "info!" in j or "debug!" in j or "trace!" or "error!" in j:
                j = j.replace("warn!","println!").replace("trace!","println!").replace("info!","println!").replace("debug!","println!")
                j = j.replace("error!","println!")
              if "HashMap::default()" in j:
                j = j.replace("HashMap::default()","HashMap::<_, _, RandomState>::default()")
              if "AHasher:default()" in j:
                j = j.replace("AHasher:default()","DefaultHasher::default()")
              if "impl_println" in j:
                filter_out = True
              if "pub use crate::reflect::ReflectComponent;" in j:
                filter_out = True
              if "Unique mutable borrow of a Reflected component" in j:
                filter_out = True
              if '#[cfg(feature = "bevy_reflect")]' in j:
                filter_out = True
                watch_restricted = True
                restricted = "{"
                print("set restricted 1..")
              # if "use bevy_reflect" in j:
              #   filter_out = True
              #   restricted = 4
              # if "use std::collections::hash_map::Entry;"
              elif restricted=="{":
                if "{" not in j:
                  print("j",j)
                  filter_out = True
                  watch_restricted = False
                  print("set restricted ..")
                  restricted=""
                else:
                  print("h",j,j.endswith(restricted+"\n"))
              if restricted in j and restricted!="":
                watch_restricted = False
                filter_out = True
                print("z",j,j.endswith(restricted+"\n"),"restricted",restricted)
                if restricted=="{":
                  watch_restricted = True
                  restricted="}"
                else:
                  print("set_restricted")
                  restricted = ""
              if watch_restricted:
                filter_out = True
              if filter_out ==False:
                w.write(j)
