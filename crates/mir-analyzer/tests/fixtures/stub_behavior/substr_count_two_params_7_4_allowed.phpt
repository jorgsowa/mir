===description===
substr count two params 7 4 allowed
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
$output = 'some text with info here';
// In PHP 7.4+, substr_count accepts 2 parameters (offset/length are optional)
$count = substr_count($output, 'info');
