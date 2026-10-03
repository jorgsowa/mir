===description===
$arr['k'] .= 'x' analyzes the array-offset target instead of being treated as an opaque string.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run(): void {
    $arr = ['k' => 'a'];
    $arr['k'] .= 'b';
    $x = $arr['k'];
    /** @mir-check $x is 'ab' */
    $_ = $x;
}
===expect===
