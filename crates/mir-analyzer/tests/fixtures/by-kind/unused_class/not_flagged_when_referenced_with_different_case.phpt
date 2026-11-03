===description===
Class names are case-insensitive: `new casemismatch()` references `CaseMismatch`
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class CaseMismatch {}

new casemismatch();
//  ^^^^^^^^^^^^ WrongCaseClass: Class name 'casemismatch' has incorrect casing; use 'CaseMismatch'
