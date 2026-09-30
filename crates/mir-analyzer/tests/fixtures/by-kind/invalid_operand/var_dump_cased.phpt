===description===
Var dump cased
===file===
<?php
vAr_dUMp("hello");
//<^^^^^^^^^^^^^^^^^ ForbiddenCode: Use of vAr_dUMp is forbidden
//<^^^^^^^^ WrongCaseFunction: Function name 'vAr_dUMp' has incorrect casing; use 'var_dump'
===expect===
