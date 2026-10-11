Hungarian spelling dictionary (hu-hu.aff, hu-hu.dic)
====================================================

Magyar Ispell 1.9 (December 2025), Hungarian Hunspell dictionary with morphological data.
(c) 2002-2025 László Németh and Ferenc Godó. Home page: http://magyarispell.sf.net

Source: the LibreOffice dictionaries repository, hu_HU/hu_HU.aff and hu_HU/hu_HU.dic,
https://github.com/LibreOffice/dictionaries/tree/32b006a2c22a4ac7e8ed3f03346f7b3d85a970a4/hu_HU
  hu_HU.aff  SHA-256 f3a2748dd535cfde2142ab17d0f7f8e4787b03fb25a60829c69ac8d493db4802
  hu_HU.dic  SHA-256 97293d670ad4a3b8e7eebef7e25c6e8e939b914c64b6b4672b2bf416b768f990

Licence: the authors offer the files under the Mozilla Public License 2.0 or the GNU Lesser
General Public License 3.0 or later, at the recipient's choice. WordCraft uses and redistributes
them under the Mozilla Public License 2.0 (MPL-2.0.txt in this folder). The files are not covered
by WordCraft's MIT OR Apache-2.0 licence.

Changes made for WordCraft (hu-hu.aff only; hu-hu.dic is unchanged):
- The original file is UTF-8 text with single-byte affix flags. Each flag byte was re-encoded as
  the Unicode character with the same number (byte 0xNN -> U+00NN) and "FLAG UTF-8" was added
  after "SET UTF-8", so the whole file is valid UTF-8. Flag identity, rules and word data are
  unchanged.
- The Latin-2 comment header and the NAME line were converted to UTF-8.

The Source Code Form of these files is the files themselves, in this folder.
