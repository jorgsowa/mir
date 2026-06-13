===description===
Inside isset disabled for dim
===config===
suppress=MixedArrayAccess,MixedArrayOffset
===file===
<?php
isset($a[$b]);
===expect===
