===description===
Amounts clamped with `min()`, compared with a bound first, or from the program are
fine.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function render(int $depth): void {
    $padding = str_repeat(' ', min((int) $_GET['width'], 200));
    $count = (int) $_POST['count'];
    if ($count > 100) {
        return;
    }
    $pages = range(1, $count);
    $delay = (int) $_GET['delay'];
    if ($delay >= 0 && $delay <= 5) {
        sleep($delay);
    }
    $indent = str_repeat('  ', $depth);
    $items = array_fill(0, count((array) $_POST['items']), 0);
}
