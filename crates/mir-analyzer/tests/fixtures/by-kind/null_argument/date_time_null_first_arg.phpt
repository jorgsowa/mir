===description===
Date time null first arg
===file===
<?php
$date = new DateTime(null);
===expect===
NullArgument@2:22-2:26: Argument $datetime of DateTime::__construct() cannot be null
