===description===
Var dump cased
===file===
<?php
vAr_dUMp("hello");
===expect===
ForbiddenCode@2:1-2:18: Use of vAr_dUMp is forbidden
WrongCaseFunction@2:1-2:9: Function name 'vAr_dUMp' has incorrect casing; use 'var_dump'
