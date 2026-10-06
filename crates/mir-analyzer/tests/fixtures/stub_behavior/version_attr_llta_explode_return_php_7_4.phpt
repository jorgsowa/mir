===description===
LanguageLevelTypeAware return: explode() returns string[]|false (default) on PHP 7.4
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyInvalidArrayAccess errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>7.4</phpVersion>
</mir>
===file===
<?php
$parts = explode(",", "a,b,c");
/** @mir-check $parts is non-empty-list<string>|false */
echo $parts[0];
