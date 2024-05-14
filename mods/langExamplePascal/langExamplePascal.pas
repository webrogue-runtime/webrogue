library langExamplePascal;

Uses strings, sysutils;

type
  langExampleFunc =  procedure();

procedure fpc_libinitializeunits(); stdcall; external;

procedure addLangExample(const name: PChar; func: langExampleFunc); stdcall; external;
procedure langExampleReturn(const name: PChar); stdcall; external;
procedure webrogue_core_print(const s: PChar); stdcall; external;

procedure pascalLangExample();
var
  str: string;
  c_str: PChar;
begin
  str := 'Hello to ' + IntToStr(random(99)) + ' worlds on Pascal!';
  c_str := StrAlloc(length(str)+1);
  StrPCopy(c_str, str);
  langExampleReturn(c_str);
  StrDispose(c_str);
end;

procedure init_mod_langExamplePascal();cdecl; alias : 'init_mod_langExamplePascal'; 
begin
  // webrogue_core_print('aaaaaa?');
  addLangExample('Pascal language using FreePascal', @pascalLangExample);
  fpc_libinitializeunits;
  WriteLn ('Length of 1212'); 
end;

end.
