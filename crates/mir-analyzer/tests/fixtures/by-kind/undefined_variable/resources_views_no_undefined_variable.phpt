===description===
UndefinedVariable is suppressed for files under resources/views/ path
where variables come from the view composer / template engine
===file:resources/views/dashboard.php===
<?php
echo $user;
echo $notifications;
echo $settings;
===expect===
