===description===
Returning the name of an ordinary class from `@return enum-string` or `@return trait-string` is reported.
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

interface Shape {}

final class Circle implements Shape {}

/** @return enum-string */
function notAnEnum()
{
    return Circle::class;
//  ^^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'class-string<App\Circle>' is not compatible with declared 'enum-string'
}

/** @return trait-string */
function notATrait()
{
    return Shape::class;
//  ^^^^^^^^^^^^^^^^^^^^ InvalidReturnType: Return type 'class-string<App\Shape>' is not compatible with declared 'trait-string'
}
