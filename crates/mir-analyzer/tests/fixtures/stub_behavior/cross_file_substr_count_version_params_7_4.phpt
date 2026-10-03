===description===
cross file substr count version params 7 4
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>7.4</phpVersion>
</mir>
===file:StringHelper.php===
<?php
function countWord(string $output): int {
    // In PHP 7.4+, substr_count accepts 2 parameters (offset/length are optional)
    return substr_count($output, 'info');
}
===file:App.php===
<?php
$result = countWord('some text with info here');
===expect===
