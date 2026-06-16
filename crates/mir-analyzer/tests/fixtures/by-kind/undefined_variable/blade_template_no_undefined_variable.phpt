===description===
UndefinedVariable is suppressed in .blade.php view template files where
variables are injected by the template engine from the calling scope
===file:resources/views/greeting.blade.php===
<?php
echo $title;
echo $userName;
===expect===
