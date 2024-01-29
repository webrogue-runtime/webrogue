cmake_policy(SET CMP0007 NEW)

function(create_resources resource)
    string(REGEX MATCH "([^/]+)$" filename ${resource})
    string(REGEX REPLACE "\\.| |-" "_" filename ${filename})
    file(READ ${resource} filedata HEX)
    string(REGEX REPLACE "([0-9a-f][0-9a-f])" "0x\\1, " filedata ${filedata})

    file(WRITE embedded_resources/${filename}.c "const unsigned char ${filename}[] = {${filedata}};\nconst unsigned ${filename}_size = sizeof(${filename});\n")
    file(WRITE embedded_resources/${filename}.h "#ifdef __cplusplus\nextern \"C\" {\n#endif\nextern const unsigned char ${filename}[];\nextern const unsigned ${filename}_size;\n#ifdef __cplusplus\n}\n#endif\n")
endfunction()

create_resources(${INPUT_FILE})
