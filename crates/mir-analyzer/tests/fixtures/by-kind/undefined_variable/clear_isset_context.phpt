===description===
Clear isset context
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function greet(bool $arg): ?string
{
    return $arg ? "hi" : null;
}

echo greet($undef) ?? "bye";
===expect===
