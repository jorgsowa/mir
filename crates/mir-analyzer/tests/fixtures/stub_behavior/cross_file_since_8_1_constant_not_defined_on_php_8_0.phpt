===config===
php_version=8.0
===file:ImageHelper.php===
<?php
function is_avif(int $type): void {
    echo ($type === IMAGETYPE_AVIF ? 'avif' : 'other');
}
===file:App.php===
<?php
is_avif(19);
===expect===
ImageHelper.php: UndefinedConstant: Constant IMAGETYPE_AVIF is not defined
