'use strict';
import * as langExampleCore from 'langExampleCore';

var selfFunc

function callback() {
    let result = "QuickJS example.\n" + selfFunc.toString() + "\n" + Math.random()
    langExampleCore.langExampleReturn(result)
}
selfFunc = callback

langExampleCore.addLangExample(
    'QuickJS example',
    callback
);
