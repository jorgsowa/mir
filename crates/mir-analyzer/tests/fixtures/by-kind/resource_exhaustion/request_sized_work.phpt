===description===
A request value, even cast to `int`, sizing a repeat, array or delay lets one
request use as much memory or time as the client asks for.
===config===
<mir>
  <issueHandlers>
    <ArgumentTypeCoercion errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function render(): void {
    $padding = str_repeat(' ', (int) $_GET['width']);
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: str_repeat() takes its size, count or duration straight from the request — possible denial of service
    $count = intval($_POST['count']);
    $pages = range(1, $count);
//           ^^^^^^^^^^^^^^^^ ResourceExhaustion: range() takes its size, count or duration straight from the request — possible denial of service
    $slots = array_fill(0, $count, null);
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: array_fill() takes its size, count or duration straight from the request — possible denial of service
    sleep((int) $_GET['delay']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ResourceExhaustion: sleep() takes its size, count or duration straight from the request — possible denial of service
}
