from genericpath import exists
import hashlib
import os
from discord import *
import re
import requests
import json

class LocalClient(Client):
    def __init__(self, *, loop=None, **options):
        super().__init__(intents = Intents.all(),loop=loop, **options)
        
        try:
            config = json.load(open("./config.json"))
            self.fetchSize: int = config["max_fetch_files"]
            self.showUnhandled = config["show_unhandled_messages"] == "True"
            self.exitOnError = config["exit_on_error"] == "True"
            self.downloadFolderPath = config["download_folder_path"]
            self.validFilesRegex = re.compile(config["valid_files"])
            self.showSkips = config["show_skips"] == "True"
            self.exitAfterCommand = config["exit_after_command"] == "True"
        except IOError as ioErr:
            print("error > {0}".format(ioErr))
            self.showUnhandled = False
            self.fetchSize = -1

        self.name = ""
        self.command = re.compile(r"^!get-images")
        self.resourceRegex = re.compile(r"(https?: \/\/(?:www\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]\.[^\s]{2,}|www\.[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]\.[^\s]{2,}|https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9]+\.[^\s]{2,}|www\.[a-zA-Z0-9]+\.[^\s]{2,})", re.IGNORECASE)
        self.fileExtensionRegex = re.compile(r'\.[A-z0-9]+$')


    async def on_ready(self):
        print("bot > logged in as {0}, waiting for command".format(self.user.display_name))


    async def on_message(self, message: Message):
        print("bot > {0}: {1}".format(message.author.display_name, message.clean_content))

        try:
            if self.command.match(message.clean_content):
                print("bot > command recognized, fetching messages.")
                await self._handleCommand(message)
            elif self.showUnhandled:
                print("bot > \"{0}\" command not recognized.".format(message.clean_content))
        except Exception as ex:
            print("error > Exception caught in on_message.\n{0}".format(ex))
            if (self.exitOnError):
                print("Exiting program.")
                exit(-1)
        

    async def _handleCommand(self, message: Message):
        channel: TextChannel = message.channel
        messages = None

        if self.fetchSize < 0:
            print("bot > \tfetching messages in {0}, this might take a while...".format(channel.name))
            messages = [message async for message in channel.history(limit = None)]
        else:
            print("bot > \tfetching messages (maximum: {0}) in {1}, this might take a while...".format(self.fetchSize, channel.name))
            messages = [message async for message in channel.history(limit = self.fetchSize)]

        # print("bot > \t\tfetched {0} messages, searching for images.".format(len(messages)))

        filteredMessages = []
        for message in messages:
            if (len(message.attachments) > 0):
                for attachement in message.attachments:
                    filteredMessages.append(attachement.url)
            else:
                matches = self.resourceRegex.match(message.clean_content)
                if matches:
                    filteredMessages.append(matches[0]) 

        print("bot > \t\tfound {0} valid messages.".format(len(filteredMessages)))
        if not self._checkDownloadFolder():
            print("bot > created download folder.")
        
        print("bot > \tdownloading to : {0}".format(self.downloadFolderPath))
        i = 0
        downloads = 0
        max = len(filteredMessages)
        for url in filteredMessages:
            i += 1

            if self.validFilesRegex.search(url):
                print("bot > \t\t[{0}/{1}] downloading {2}".format(i, max, url))
                res = requests.get(url)

                if res.ok:
                    bs = res.content # binary content
                    fileName = hashlib.sha256(url.encode("utf-8")).hexdigest() + self._getFileExtension(url)
                    print("bot > \t\t\twriting content to {0}".format(fileName))
                    try:
                        if not exists("{0}{1}".format(self.downloadFolderPath, fileName)):
                            fp = open("{0}{1}".format(self.downloadFolderPath, fileName), "xb")
                            fp.write(bs)
                            fp.close()
                            downloads = downloads + 1
                        else:
                            print("bot > \t\t\tskipping {0}, file already exists".format(fileName))
                    except Exception as ex:
                        print("error > {0}".format(ex))
                else:
                    print("bot > \t\t\tfailed to GET {0}".format(url))

            elif self.showSkips:
                print("bot > \t\t[{0}/{1}] skipping {2}, not matching valid files regex.".format(i, max, url))
        
        print("bot > \tdownloaded {0} files to {1}.".format(downloads, self.downloadFolderPath))
        if self.exitAfterCommand:
            print("bot > a job well done ! Bye bye ! *^____^*")
            self.close()
            exit(1)


    def _checkDownloadFolder(self) -> bool:
        if not exists(self.downloadFolderPath):
            os.mkdir(self.downloadFolderPath)
            return False
        else:
            return True

    def _getFileExtension(self, s: str) -> str:
        match = self.fileExtensionRegex.search(s)
        if match:
            return match[0]
        else:
            return ""