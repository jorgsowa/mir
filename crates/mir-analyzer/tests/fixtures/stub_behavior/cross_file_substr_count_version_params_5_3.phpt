===description===
cross file substr count version params 5 3
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>5.3</phpVersion>
</mir>
===file:StringHelper.php===
<?php
function countWord(string $output): int {
    // In PHP 5.3, substr_count required offset parameter
    return substr_count($output, 'info', 0);
}
===file:App.php===
<?php
$result = countWord('some text with info here');
