===description===
IntlTimeZone helpers whose trailing parameters are optional at runtime can be called without them.
===file===
<?php
function zones(IntlTimeZone $tz): string|false {
    intltz_create_enumeration();
    intltz_get_canonical_id('Europe/Paris');
    return intltz_get_display_name($tz);
}
