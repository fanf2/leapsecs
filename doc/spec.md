Compact formats for the Leap Second list
========================================

_Tony Finch_
_May 2021_


abstract
--------

This memo specifies two compact formats for the leap second list: a
simple text format that uses about 3 characters per leap second; and a
very small binary format that uses about 5 bits per leap second. The
aim is to make it very cheap to distribute the full leap second list
alongside the current time.


standards and procedures
------------------------

[ITU recommendation TF.460-6][TF.460-6] Annex I specifies UTC and leap
seconds.

DTAI is the difference between TAI and UTC. A positive leap second
increases DTAI by one, and a negative leap second decreases DTAI by
one. (So far there have been no negative leap seconds.)

A leap second can occur at the end of any month (of the Gregorian
calendar, though that is not explicitly specified by [TF.460-6][]).
First preference is that leap seconds should occur at the end of June
or December. (So far there have been no leap seconds in other months.)

[TF.460-6]: http://www.itu.int/rec/R-REC-TF.460-6-200202-I


### existing formats

Leap second tables are distributed by the [IERS][] and [NIST][]. They
use different formats, but they contain similar information.

Each leap second is indicated by the date immediately following, i.e.
the first of the month (so far, always January and July.), and the
value of DTAI starting at that date.

The tables do not explicitly say whether a leap second is positive or
negative; that is implied by the difference between successive values
of DTAI.

The first entry in the tables is for 1 Jan 1972 when DTAI became 10
seconds. That was the start of UTC rather than a leap second.

The tables also include an expiry date.

[IERS]: https://hpiers.obspm.fr/eoppc/bul/bulc/Leap_Second.dat
[NIST]: ftp://time.nist.gov/pub/leap-seconds.list


### publication and expiry

The usual practice is the IERS will issue [Bulletin C][BulC] in
January and July each year, to announce whether or not there will be a
leap second at the end of June or December (respectively).

The [IERS][] and [NIST][] leap second tables are updated soon after.
January issues have expiry dates of 28th December the same year, and
July issues have an expiry date of 28th June the following year.

So the validity period for the leap second tables is roughly 11
months.

[BulC]: https://datacenter.iers.org/availableVersions.php?id=16


compact formats
---------------

Both the compact text and binary formats are based on the same
principles.

  * The gap between leap seconds is given as a count of months.

    Months correspond exactly to the [TF.460-6][] requirements; higher
    resolution times would waste space. Relative time periods can be
    more compact than the absolute dates used by existing formats.

  * Each leap second is marked as positive or negative.

    DTAI is not given explicitly; instead, a reader can calculate it
    by accumulating the positive and negative leap seconds.

  * The expiry date is rounded down to the start of the month.

    This is so that the expiry date can be written at the end of the
    list in a similar way to the leap seconds.


text format
-----------

Here is an example of the compact text format. It list the 19 leap
seconds through the Bulletin C issued in January 1994, announcing the
leap second at the end of June 1994, and expiring in December 1994.

        6+6+12+12+12+12+12+12+12+18+12+12+24+30+24+12+18+12+12+5?

The [ABNF][] for the text format is:

            leaps       = *leap end

            leap        = gap sign

            sign        = "+" / "-"

            end         = gap "?"

            gap         = nonzero *2digit

            digit       = "0" / nonzero

            nonzero     = %x31-39
                            ; 1 - 9

[ABNF]: https://tools.ietf.org/html/rfc5234

The compact text leap second list is a sequence of numbers, giving the
gap between leap seconds counted in months. The numbers are separated
by leap indicators and terminated by an expiry indicator.

Each gap is a decimal number between 1 and 999 without leading zeroes.

(Fewer than three digits is not enough to represent gaps that might
occur; three digits allows for 83 years, which is expected to be more
than enough; more than three digits would cause problems for the
binary format.)

A negative leap second is indicated by a '-' and a positive leap
second by a '+'.

The last number is the number of months between the last leap second
and the expiry of the list, rounded down to a whole number of months.
The list is terminated with a '?' expiry indicator.

