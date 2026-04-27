===config===
find_dead_code=true
===file===
<?php
namespace App;

function helper(): void {}

helper();
===expect===
