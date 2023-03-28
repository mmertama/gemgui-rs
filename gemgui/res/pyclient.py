import os
import asyncio
import json
from urllib.parse import urlparse
import sys
import webview
import websockets
import re
import argparse
import ast

'''
import logging
logger = logging.getLogger('websockets')
logger.setLevel(logging.DEBUG)
logger.addHandler(logging.StreamHandler())
'''

do_exit = None

# file_types = ('Image Files (*.bmp;*.jpg;*.gif)', 'All files (*.*)')
def make_filters(filters):
    if not filters:
        return tuple()
    filters_list = []
    for k, f in filters.items():
        filter_string = '{} ({})'.format(k, ';'.join(f))
        filters_list.append(filter_string)
    return tuple(filters_list)

def open_file_dialog(window, params, ex_id):
    dir_name = params['dir']
    filters = params['filters']

    result = window.create_file_dialog(webview.OPEN_DIALOG,
                                        directory=dir_name,
                                        allow_multiple=False,
                                        file_types=make_filters(filters))
    response = json.dumps({
        'type': 'extension_response',
        'extension_call': 'openFileResponse',
        'extension_id': ex_id,
        'openFileResponse': str(result[0]) if result else ''})
    return response
        

def open_files_dialog(window, params, ex_id):
    dir_name = params['dir']
    filters = params['filters']
    result = window.create_file_dialog(webview.OPEN_DIALOG,
                                        directory=dir_name,
                                        allow_multiple=True,
                                        file_types=make_filters(filters))
    response = json.dumps({
        'type': 'extension_response',
        'extension_call': 'openFilesResponse',
        'extension_id': ex_id,
        'openFilesResponse': list(result) if result else []})
    return response
        

def open_dir_dialog(window, params, ex_id): 
    dir_name = params['dir']
    result = window.create_file_dialog(webview.FOLDER_DIALOG,
                                        directory=dir_name,
                                        allow_multiple=False)
    response = json.dumps({
        'type': 'extension_response',
        'extension_call': 'openDirResponse',
        'extension_id': ex_id,
        'openDirResponse': str(result[0]) if result else ''})
    return response


def save_file_dialog(window, params, ex_id):
    dir_name = params['dir']
    filters = params['filters']
    result = window.create_file_dialog(webview.SAVE_DIALOG,
                                        directory=dir_name,
                                        allow_multiple=False,
                                        file_types=make_filters(filters))
    response = json.dumps({
        'type': 'extension_response',
        'extension_call': 'saveFileResponse',
        'extension_id': ex_id,
        'saveFileResponse': str(''.join(result)) if result else ''})
    return response
    

def resize(window, params):
     # window.resize include titlebar, so we get current body and get title height, so we can add it to get requested body size
    # known issue, does not work well with FRAMELESS. Fix someday.
    vp_height = window.evaluate_js(r'Math.min(window.innerHeight, document.documentElement.clientHeight);')
    vp_width = window.evaluate_js(r'Math.min(window.innerWidth, document.documentElement.clientWidth);')
    border_height = window.height - vp_height
    border_width = window.width - vp_width
    width = params['width']
    height = params['height']
    window.resize(width + border_width, height + border_height)


def set_title(window, params):
    title = params['title']
    window.set_title(title)

def on_show(window, host, port):
    ws_uri = 'ws://{}:{}/gemgui/extension'.format(host, port)
    window_destroyed = False

    async def extender():    
        async with websockets.connect(ws_uri,
                                    close_timeout=None,
                                    ping_interval=None,
                                    compression=None) as ws:
            nonlocal window_destroyed
            loop = asyncio.get_event_loop()
            #receive = loop.create_task(ws.recv())
            
            def destroy_window():
                if not window_destroyed:
                    window.minimize()  # it takes some time
                    window.destroy()
                return
            
            

            def exit_f():
                pass
                ##loop.run_until_complete(ws.close)
                #nonlocal receive
                #receive.cancel()

            global do_exit
            do_exit = exit_f

            try:
                await ws.send(json.dumps({'type': 'extensionready'}))
            except Exception as e:
                print(f"Initial send failed {e}") 
                return   

            while True:
                receive = loop.create_task(ws.recv())
                try:
                    await receive
                except asyncio.CancelledError:
                    destroy_window()
                    await ws.close()
                    return
                except websockets.ConnectionClosedError as e:
                    print(f"Connection closed: {ws_uri} due {e}")
                    destroy_window()
                    return

                doc = receive.result()

                if(not isinstance(doc, str)):
                    continue

                
                try:
                    obj = json.loads(doc)
                except UnicodeDecodeError as e:
                    print('UnicodeDecodeError on extender:', e, '\nWhen parsing:', doc[:1000], file=sys.stderr)
                    return
                except json.decoder.JSONDecodeError as e:
                    if(doc == "entered"): 
                        continue # gemgui internal 
                    print('JSONDecodeError on extender:', e, '\nWhen parsing:', doc[:1000], file=sys.stderr)
                    return 
                
                if not type(obj) is dict:
                    print('Invalid JS object', doc[:1000])
                    continue

                if obj['type'] == 'exit_request' or obj['type'] == 'close_request':
                    window_destroyed = True
                    window.destroy()
                    #ws.close()
                    # loop.stop()
                    #return
                    continue

                if obj['type'] != 'extension':
                    continue

                call_id = obj['extension_call']
                params = obj['extension_params']
               
                response = None
                
                if call_id == 'setAppIcon':
                    pass
                
                if call_id == 'resize':
                    resize(window, params)
                   
                if call_id == 'setTitle':
                   set_title(window, params)

                if call_id == 'ui_info':
                    pass

                ex_id = obj['extension_id']    
                if call_id == 'openFile':
                    response = open_file_dialog(window, params, ex_id)

                if call_id == 'openFiles':
                    response = open_files_dialog(window, params, ex_id)

                if call_id == 'saveFile':
                    response = save_file_dialog(window, params, ex_id)

                if call_id == 'openDir':
                    response = open_dir_dialog(window, params, ex_id)            


                if response:
                    await ws.send(response)

    asyncio.run(extender())

def on_close():
    if do_exit:
        do_exit()
    os._exit(0) # pyvwebview is very slow to close sockets.



def main():
    width = 1024
    height = 768
    title = ''
    extra = {}

    NORESIZE = 0x1
    FULLSCREEN = 0x2
    HIDDEN = 0x4
    FRAMELESS = 0x8
    MINIMIZED = 0x10
    ONTOP = 0x20
    CONFIRMCLOSE = 0x40
    TEXTSELECT = 0x80
    EASYDRAG = 0x100
    TRANSPARENT = 0x200

    flags = 0

    parser = argparse.ArgumentParser()
    parser.add_argument('--gempyre-url', type=str)
    parser.add_argument('--gempyre-width', type=int)
    parser.add_argument('--gempyre-height', type=int)
    parser.add_argument('--gempyre-title', type=str)
    parser.add_argument('--gempyre-extra', type=str)
    parser.add_argument('--gempyre-flags', type=int)
   # parser.add_argument('url', type=str)
    parser.add_argument('-c', action='store_true') # clean off

    try:
        args = parser.parse_args()
    except argparse.ArgumentError:
        pass

    if args.gempyre_width:
        width = int(args.gempyre_width)

    if args.gempyre_height:
        height = int(args.gempyre_height)

    if args.gempyre_title:
        title = args.gempyre_title

    if args.gempyre_url:
        uri_string = args.gempyre_url
    elif args.url:
        uri_string = args.url

    if args.gempyre_flags:
        flags = args.gempyre_flags

    if sys.platform == 'win32':
        extra['gui'] = 'cef'

    if args.gempyre_extra:
        for e in args.gempyre_extra.split(';'):
            m = re.match(r'^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.*)\s*$', e)
            ex_name = m[1]
            e_lit = m[2]
            try:
                ex_value = ast.literal_eval(e_lit)
                extra[ex_name] = m[ex_value]
            except ValueError as err_s:
                print("Invalid parameter in", e_lit, "of", e, ": ", err_s)

    uri = urlparse(uri_string)
    window = webview.create_window(title, url=uri_string, width=width, height=height,
    resizable = True if not flags & NORESIZE else False,
    fullscreen = True if flags & FULLSCREEN else False,
    hidden = True if flags & HIDDEN else False,
    frameless = True if flags & FRAMELESS else False,
    minimized = True if flags & MINIMIZED else False,
    on_top = True if flags & ONTOP else False,
    confirm_close = True if flags & CONFIRMCLOSE else False,
    text_select = True if flags & TEXTSELECT else False,
    easy_drag = True if flags & EASYDRAG else False,
    transparent = True if flags & TRANSPARENT else False)
    if hasattr(window, 'events'): # version compliancy
        window.events.shown += lambda: on_show(window, uri.hostname, uri.port)
        window.events.closing += on_close
    else:
        window.shown += lambda: on_show(window, uri.hostname, uri.port)
        window.closing += on_close
    webview.start(**extra)

    
if __name__ == '__main__':
    main()


