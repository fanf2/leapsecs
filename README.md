Compact formats for the leap second list
========================================

<img src="doc/logo.png" width="50%" align="right" >

The goal is to make it really easy to distribute the list of leap
seconds, by making the list really small and easy to read and write.

The binary leap second list is smaller than typical cryptographic
authentication codes, so it should be cheap enough for time servers to
include it in most responses.

In the first half of 2021, the text format looks like this. (The line
break is not part of the format; it's just for presentation purposes
in this README.) It is currently 82 characters; I expect it will
remain less than 100 characters for the next 20 years.

        6+6+12+12+12+12+12+12+12+18+12+12+24+30+24+
        12+18+12+12+18+18+18+84+36+42+36+18+59?

The binary format looks like this. (Represented as a hex dump rather
than raw binary.) It is currently 16 bytes, and may grow past 20 bytes
in the next 20 years.

        00111111 12113431 2112229D 565287FA

These leap second lists are cryptographically signed and published in
the DNS at `leapsecond.dotat.at`.


spec
----

The specification for the compact leap second list text and binary
formats can be found in [doc/spec.md](doc/spec.md).


code
----

This repository contains a Rust library and program for reading and
writing the leap seconds list in various formats, including:

  * a compact text format
  * a compact binary format
  * the NIST `leap-seconds.list` format

The features implemented by the library are reasonably complete,
though there arelots of missing features (see the todo list below).

The example program is minimal: it just downloads the NIST
`leap-seconds.list` or reads a cached copy from a file, and prints the
leap secons list in regenerated NIST format, in compact text format,
and a hex dump of the compact binary format.


todo
----

I have some old prototype tools that could do with being replaced. To
be a replacement, this code needs to:

  * read and write the IERS leap second list published at
    https://hpiers.obspm.fr/eoppc/bul/bulc/

  * DNS query and update support for reading and writing the list at
    `leapsecond.dotat.at` or elsewhere;

  * read and write the leap second list in DNS AAAA records, as BCD
    encoded ISO 8601 timestamps (and write a spec for this format)

  * read and write leap second list in DNS A records, in the style of
    Poul-Henning Kamp, http://phk.freebsd.dk/time/20151122/

There are open questions about how to make these leap second lists
useful for time distribution in general. One possibility is so that
NTP clients and servers can ensure they have correct leap second
information, as a cross-check for NTP's leap indicator bits.


licence
-------

> This was written by Tony Finch <<dot@dotat.at>>  
> You may do anything with it. It has no warranty.  
> <https://creativecommons.org/publicdomain/zero/1.0/>  
> SPDX-License-Identifier: CC0-1.0
