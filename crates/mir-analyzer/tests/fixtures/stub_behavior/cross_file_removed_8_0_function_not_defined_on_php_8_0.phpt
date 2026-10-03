===description===
cross file removed 8 0 function not defined on php 8 0
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file:TextHelper.php===
<?php
function format_hebrew(string $text): void {
    hebrevc($text);
//  ^^^^^^^^^^^^^^ UndefinedFunction: Function hebrevc() is not defined
}
===file:App.php===
<?php
format_hebrew('שלום');
===expect===
