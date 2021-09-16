# SPDX-FileCopyrightText: 2020 Jonah Brüchert <jbb@kaidan.im>
#
# SPDX-License-Identifier: AGPL-3.0-only

json.dumps(youtube_dl.YoutubeDL(options).extract_info(url, download=False))
