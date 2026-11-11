===description===
A function declared `@return numeric` may return a numeric string literal.
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

/** @return numeric */
function fromList()
{
    $values = [1, 1.5, '123'];

    return $values[\array_rand($values)];
}

/** @return numeric */
function notNumeric()
{
    return 'abc';
//  ^^^^^^^^^^^^^ InvalidReturnType: Return type '"abc"' is not compatible with declared 'numeric'
}
