===description===
cross file since 8 1 function not defined on php 8 0
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file:ArrayHelper.php===
<?php
function check_is_list(array $items): void {
    array_is_list($items);
//  ^^^^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function array_is_list() is not defined
}
===file:App.php===
<?php
check_is_list([1, 2, 3]);
