import React from 'react';
import { Routes, Route, Link, useParams } from 'react-router-dom';
import Sidebar from '../components/Sidebar';
import '../styles/pages/HomePage.css';
import FriendList from './home/FriendList';
import MessageList from './home/MessageList';
import ChatWindow from './home/ChatWindow';

const HomePage: React.FC = () => {
  const { page } = useParams<{ page: string }>();

  return (
    <div className="home-page">
      <Sidebar />
      <div className="content">
        <Routes>
          <Route path="friendlist" element={<FriendList />} />
          <Route path="messagelist" element={<MessageList />} />
          <Route path="chat/:userId" element={<ChatWindow />} />
          <Route path="/" element={<div>请选择一个页面</div>} />
        </Routes>
      </div>
    </div>
  );
};

export default HomePage;
