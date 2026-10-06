===description===
cross file since 8 1 constant not defined on php 8 0
===config===
<mir>
  <phpVersion>8.0</phpVersion>
</mir>
===file:ImageHelper.php===
<?php
function is_avif(int $type): void {
    echo ($type === IMAGETYPE_AVIF ? 'avif' : 'other');
//                  ^^^^^^^^^^^^^^ UndefinedConstant: Constant IMAGETYPE_AVIF is not defined
}
===file:App.php===
<?php
is_avif(19);
