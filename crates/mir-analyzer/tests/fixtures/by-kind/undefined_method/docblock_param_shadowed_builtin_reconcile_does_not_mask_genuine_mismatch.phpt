===description===
P28 control: the docblock-shadowed-builtin reconciliation (see the P28
regression fixtures) only ever narrows a bare *builtin* name
(`Generator`/`Closure`/etc.) to a same-namespace local class of that exact
name. A `@param` naming a genuinely different, unrelated class must still
store that class's own type — and calling a method that class doesn't have
must still be a real, caught `UndefinedMethod`.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

final class Other {}

/**
 * @param Other $g
 */
function useIt(Generator $g): string
//                       ^^ MismatchingDocblockParamType: Docblock type 'App\Other' for $g does not match inferred 'App\Generator'
{
    return $g->build();
//  ^^^^^^^^^^^^^^^^^^^ MixedReturnStatement: Cannot return a mixed type from function with declared return type 'string'
//         ^^^^^^^^^^^ UndefinedMethod: Method App\Other::build() does not exist
}
===expect===
