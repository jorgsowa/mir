===description===
LanguageLevelTypeAware return: explode() returns string[] (no false) on PHP 8.0
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.0</phpVersion>
</mir>
===file===
<?php
$parts = explode(",", "a,b,c");
/** @mir-check $parts is non-empty-list<string> */
echo $parts[0];
===expect===
