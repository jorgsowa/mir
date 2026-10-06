===description===
FALSE POSITIVE reproducer. Valid PHP: `Color::{$name}` is dynamic enum-case access on a defined enum, not a bare constant.
Expected: no issue.
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
function pick(string $name): Color {
    return Color::{$name};
}
