===description===
stub file function no error
===config===
<mir>
  <stubs>
    <file name="stubs/helpers.php"/>
  </stubs>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:stubs/helpers.php===
<?php
function my_helper(string $s): string { return $s; }
===file:App.php===
<?php
$result = my_helper('hello');
