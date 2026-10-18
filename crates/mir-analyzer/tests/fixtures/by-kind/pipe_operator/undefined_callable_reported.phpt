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
$b = 1 |> (new Text())->missing(...);
$c = 1 |> Text::missing(...);
===expect===
UndefinedFunction@4:5-4:25: Function missing_fn() is not defined
UndefinedMethod@5:5-5:36: Method Text::missing() does not exist
UndefinedMethod@6:5-6:28: Method Text::missing() does not exist
