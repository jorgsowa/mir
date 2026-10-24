===description===
An undefined function or method named on the right of `|>` is reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Text {}

$a = 1 |> missing_fn(...);
//   ^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function missing_fn() is not defined
$b = 1 |> (new Text())->missing(...);
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Text::missing() does not exist
$c = 1 |> Text::missing(...);
//   ^^^^^^^^^^^^^^^^^^^^^^^ UndefinedMethod: Method Text::missing() does not exist
