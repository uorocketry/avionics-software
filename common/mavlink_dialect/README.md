## PREREQUISITES
1. Must install mavlink-bindgen
    `cargo install mavlink-bindgen --features cli`


## HOW TO GENERATE MESSAGES
1. Add the message to the uorocketry.xml file
2. Generate new code with `mavlink-bindgen .\ .\src\output`
