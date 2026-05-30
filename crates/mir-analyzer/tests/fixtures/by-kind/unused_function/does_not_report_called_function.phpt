===description===
does not report called function
===config===
suppress=
===file===
<?php
function helper(): void {}

helper();
===expect===
