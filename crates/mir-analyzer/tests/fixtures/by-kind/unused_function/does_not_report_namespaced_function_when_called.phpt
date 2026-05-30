===description===
does not report namespaced function when called
===config===
suppress=
===file===
<?php
namespace App;

function helper(): void {}

helper();
===expect===
