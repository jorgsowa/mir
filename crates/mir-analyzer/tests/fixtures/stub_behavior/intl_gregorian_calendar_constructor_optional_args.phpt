===description===
`IntlGregorianCalendar::__construct` accepts a time zone and locale, or the
date parts, and every argument is optional.
===file===
<?php
function calendars(\DateTimeZone $tz, \IntlTimeZone $itz): void {
    new \IntlGregorianCalendar();
    new \IntlGregorianCalendar($tz, \Locale::getDefault());
    new \IntlGregorianCalendar($itz);
    new \IntlGregorianCalendar('Europe/Paris', null);
    new \IntlGregorianCalendar(2024, 1, 15);
    new \IntlGregorianCalendar(2024, 1, 15, 10, 30, 0);
}

function legacy(\DateTimeZone $tz): ?\IntlGregorianCalendar {
    return intlgregcal_create_instance($tz, 'en_US');
}
