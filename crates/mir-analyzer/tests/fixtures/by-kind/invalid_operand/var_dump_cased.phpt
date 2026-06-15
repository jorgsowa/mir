===description===
Var dump cased
===file===
<?php
vAr_dUMp("hello");
===expect===
ForbiddenCode@2:0-2:17: Use of vAr_dUMp is forbidden
WrongCaseFunction@2:0-2:8: Function name 'vAr_dUMp' has incorrect casing; use 'var_dump'
