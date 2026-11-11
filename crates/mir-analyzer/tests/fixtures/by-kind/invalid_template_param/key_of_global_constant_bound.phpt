===description===
A template bounded by `key-of<GLOBAL_CONST>` accepts the keys of that constant and rejects other values.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

const ID_TABLE = [
    'immutable' => 1,
    'mutable' => 'two',
];

/**
 * @template T of key-of<ID_TABLE>
 * @param T $type
 */
function lookUp(string $type): int|string
{
    return ID_TABLE[$type];
}

lookUp('immutable');
lookUp('mutable');
lookUp('missing');
//<^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as '"missing"' does not satisfy bound 'key-of<App\ID_TABLE>'
