library langExamplePascal;

type
  langExampleFunc =  function(): PChar;

procedure fpc_libinitializeunits(); stdcall; external;

procedure addLangExample(const name: PChar; func: langExampleFunc); stdcall; external;
procedure webrogue_core_print(const s: PChar); stdcall; external;

function pascalLangExample(): PChar;
begin
  pascalLangExample := 'Hello world on Pascal?';
end;

procedure init_mod_langExamplePascal();cdecl; alias : 'init_mod_langExamplePascal'; 
begin
  // webrogue_core_print('aaaaaa?');
  addLangExample('Pascal language using FreePascal', @pascalLangExample);
  fpc_libinitializeunits;
  WriteLn ('Length of 1212'); 
end;

end.
