===description===
Laravel's `request('key')` helper reads request data like `$_GET`.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UndefinedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function pad(): void {
    echo str_repeat('-', (int) request('width'));
//       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: str_repeat() takes its size, count or duration straight from the request — possible denial of service
    $rows = (int) request('rows');
    array_fill(0, $rows, null);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: array_fill() takes its size, count or duration straight from the request — possible denial of service
}
