===description===
cross file removed 8 0 function available on php 7 4
===config===
<mir>
  <phpVersion>7.4</phpVersion>
</mir>
===file:TextHelper.php===
<?php
function format_hebrew(string $text): void {
    hebrevc($text);
}
===file:App.php===
<?php
format_hebrew('שלום');
===expect===
